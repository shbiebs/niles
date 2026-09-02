select k, sum(v) as total from t group by k order by total desc limit 2
