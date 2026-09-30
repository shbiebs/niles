select k, v, sum(v) over (partition by k order by recorded_at) as running from t
