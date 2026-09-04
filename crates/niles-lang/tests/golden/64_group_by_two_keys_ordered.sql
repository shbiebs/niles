select k, n, sum(v) from t group by k, n order by sum(v) desc limit 2
