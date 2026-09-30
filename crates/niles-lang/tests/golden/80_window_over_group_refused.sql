select k, sum(v), rank() over (order by k) from t group by k
