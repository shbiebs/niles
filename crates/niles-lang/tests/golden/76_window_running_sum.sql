select k, v, sum(v) over (order by k) as running from t
