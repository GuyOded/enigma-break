import pandas as pd
import plotly.express as px
import sys

if len(sys.argv) != 2:
    print("Usage: python plot_ioc.py <csv_file>")
    sys.exit(1)

csv_file = sys.argv[1]

df = pd.read_csv(csv_file)
df['index'] = range(len(df))

fig = px.scatter(df, x='index', y='ioc', title='IOC as function of rotor positions',
                 hover_data=['left', 'mid', 'right'])

average_ioc = df['ioc'].mean()

fig.add_annotation(text=f"Average IOC: {average_ioc:.6f}",
                   xref="paper", yref="paper",
                   x=0.5, y=-0.1,  # below the plot
                   showarrow=False,
                   font=dict(size=14))

fig.update_layout(
    xaxis_title="Index",
    yaxis_title="IOC",
    hovermode="closest"
)

fig.show()