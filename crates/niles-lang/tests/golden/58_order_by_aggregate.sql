select k, sum(v) from t group by k order by sum(v) desc limit 2
