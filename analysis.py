import pandas as pd
import numpy as np
import matplotlib.pyplot as plt

def draw_lifetime_whole(df: pd.DataFrame,pl,name:str,color:str):
    xi = list(range(len(df)))
    pl.set_title(f"Life time of whole {name}")
    pl.set_xlabel("Count")
    pl.set_ylabel("Life Time (Day)")
    pl.plot(xi,df['life_day'],color=color)

import seaborn as sns

def draw_lifetime_distribution(df: pd.DataFrame):
    sns.displot(df['life_day'],hist=True,
                bins=(180/5),color = 'dark_blue',
                hist_kws={'edgecolor':'black'},
                kde_kws={'linewidth': 4},
                )
