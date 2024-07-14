use core::panic;
use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
enum RecordType {
    Issue,
    PR,
}

#[derive(Debug, Copy, Clone, Serialize)]
enum UserType {
    Bot,
    Member,
    Contributer,
    User,
}

#[derive(Debug, Serialize)]
struct Participants {
    total: u16,
    bot: u16,
    member: u16,
    contributer: u16,
    user: u16,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
enum EndType {
    /// is merged or completed
    Green,
    /// is closed or close-as-not-a-plan
    Red,
}

#[derive(Debug, Serialize)]
pub struct Record {
    rc_ty: RecordType,
    url: String,
    author: UserType,
    closer: UserType,
    auther_is_closer: bool,

    // #[serde(flatten)]
    // participants: Participants,
    participants_total: u16,
    participants_bot: u16,
    participants_member: u16,
    participants_contributer: u16,
    participants_user: u16,

    create_time: String, // DateTime<Utc>,
    life_time_sec: i64,
    first_event_time_sec: i64,
    first_comment_time_sec: i64,
    commit_count: u64,
    comment_count: u64,
    end: EndType,
}

#[derive(Debug)]
pub enum RecordError {
    Json(serde_json::Error),
    NoCloser,
    NoTimeline,
    NoUserId,
    CreateTimeError,
    CloseTimeError,
}

impl From<serde_json::Error> for RecordError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl Record {
    pub fn parse(s: &str) -> Result<Self, RecordError> {
        let v: Value = serde_json::from_str(s)?;
        let url = v["html_url"].to_string();

        // RecordType
        let rc_ty = unsafe {
            if v["node_id"].as_str().unwrap_unchecked().starts_with("P") {
                RecordType::PR
            } else {
                RecordType::Issue
            }
        };

        // Users
        let mut user_map: BTreeMap<u64, UserType> = BTreeMap::new();

        let auther = &v["user"];
        let auther_id = unsafe { auther["id"].as_u64().unwrap_unchecked() };
        let auther_ty = if unsafe { auther["type"].as_str().unwrap_unchecked() == "Bot" } {
            UserType::Bot
        } else {
            let role = unsafe { v["author_association"].as_str().unwrap_unchecked() };
            match role {
                "CONTRIBUTOR" => UserType::Contributer,
                "Member" => UserType::Member,
                _ => UserType::User,
            }
        };
        user_map.insert(auther_id, auther_ty);

        let comment_count = *unsafe { &v["comments"].as_u64().unwrap_unchecked() };

        // time
        let created_at = v["created_at"].as_str();
        if created_at.is_none() {
            return Err(RecordError::CreateTimeError);
        }
        let created_at = created_at.unwrap();

        let create_time = DateTime::parse_from_rfc3339(created_at);
        if create_time.is_err() {
            return Err(RecordError::CreateTimeError);
        }
        let create_time = create_time.unwrap();

        // time
        let closed_at = v["closed_at"].as_str();
        if closed_at.is_none() {
            return Err(RecordError::CloseTimeError);
        }
        let closed_at = closed_at.unwrap();
        let close_time = DateTime::parse_from_rfc3339(closed_at);
        if close_time.is_err() {
            return Err(RecordError::CloseTimeError);
        }
        let close_time = close_time.unwrap();

        let life_time_sec = (close_time - create_time).num_seconds();
        let mut first_event_time_sec = -1;
        let mut first_comment_time_sec = -1;

        let timeline = v["time_line"].as_array();
        if timeline.is_none() {
            return Err(RecordError::NoTimeline);
        }
        let timeline = timeline.unwrap();

        let mut commit_count = 0;
        // 0. merged
        // 1. completed
        // 2.
        //
        //
        //
        // state reason  completed,
        //               not_planned
        // active lock reaseon  ,
        //               resolved
        //               not_planned

        let mut end_ty = EndType::Red;
        let mut closer_id: u64 = 0;

        for event in timeline.iter() {
            if first_event_time_sec == -1 {
                if let Some(event_time) = event["created_at"].as_str() {
                    let first_event_time =
                        unsafe { DateTime::parse_from_rfc3339(event_time).unwrap_unchecked() };
                    first_event_time_sec = (first_event_time - create_time).num_seconds();
                }
            }
            let ev_kind = unsafe { event["event"].as_str().unwrap_unchecked() };

            let event_user_opt = if let Some(actor) = event.get("actor") {
                Some(actor)
            } else if let Some(user) = event.get("user") {
                Some(user)
            } else {
                None
            };

            if let Some(event_user) = event_user_opt {
                if let Some(event_user_id) = event_user["id"].as_u64() {
                    user_map.entry(event_user_id).or_insert_with(|| {
                        let user_ty = if event_user["type"].as_str().unwrap() == "Bot" {
                            UserType::Bot
                        } else {
                            if let Some(role) = event["author_association"].as_str() {
                                match role {
                                    "CONTRIBUTOR" => UserType::Contributer,
                                    "MEMBER" => UserType::Member,
                                    "Member" => UserType::Member,
                                    _ => UserType::User,
                                }
                            } else {
                                UserType::User
                            }
                        };
                        user_ty
                    });
                    match ev_kind {
                        "committed" => {
                            commit_count += 1;
                        }
                        "merged" => {
                            end_ty = EndType::Green;
                            closer_id = event_user_id
                        }
                        "closed" => closer_id = event_user_id,
                        "commented" => {
                            if first_comment_time_sec == -1 {
                                let first_comment_time = unsafe {
                                    DateTime::parse_from_rfc3339(
                                        &event["created_at"].as_str().unwrap_unchecked(),
                                    )
                                    .unwrap_unchecked()
                                };
                                first_comment_time_sec =
                                    (first_comment_time - create_time).num_seconds()
                            }
                        }
                        "head_ref_force_pushed" => closer_id = event_user_id,
                        _ => {}
                    }
                } else {
                    return Err(RecordError::NoUserId);
                }
            } else {
                match ev_kind {
                    "committed" => {
                        commit_count += 1;
                    }
                    // "merged" => end_ty = EndType::Green,
                    // "commented" => {
                    //     if first_comment_time_sec == -1 {
                    //         let first_comment_time = unsafe {
                    //             DateTime::parse_from_rfc3339(
                    //                 &event["created_at"].as_str().unwrap_unchecked(),
                    //             )
                    //             .unwrap_unchecked()
                    //         };
                    //         first_comment_time_sec =
                    //             (first_comment_time - create_time).num_seconds()
                    //     }
                    // }
                    _ => {}
                }
            }
        }

        if end_ty == EndType::Red {
            if let Some(reason) = v["state_reason"].as_str() {
                match reason {
                    "completed" => end_ty = EndType::Green,
                    _ => {}
                }
            }
        }

        let participants_init = Participants {
            total: user_map.len() as u16,
            bot: 0,
            member: 0,
            contributer: 0,
            user: 0,
        };
        let participants =
            user_map
                .iter()
                .fold(participants_init, |mut p: Participants, (_uid, u_ty)| {
                    match u_ty {
                        UserType::Bot => p.bot += 1,
                        UserType::Member => p.member += 1,
                        UserType::Contributer => p.contributer += 1,
                        UserType::User => p.user += 1,
                    }
                    p
                });
        // dbg!(&v);
        // dbg!(&closer_id);
        // dbg!(&user_map);
        if closer_id == 0 {
            return Err(RecordError::NoCloser);
        }

        Ok(Self {
            rc_ty,
            url,
            author: auther_ty,
            closer: *user_map.get(&closer_id).unwrap(),
            auther_is_closer: auther_id == closer_id,
            participants_total: participants.total,
            participants_bot: participants.bot,
            participants_member: participants.member,
            participants_contributer: participants.contributer,
            participants_user: participants.user,

            create_time: create_time.to_string(),
            life_time_sec,
            first_event_time_sec,
            first_comment_time_sec,
            commit_count,
            comment_count,
            end: end_ty,
        })
    }
}
