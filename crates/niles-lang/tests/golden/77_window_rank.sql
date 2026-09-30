select k, v, rank() over (order by v desc) as r from t
