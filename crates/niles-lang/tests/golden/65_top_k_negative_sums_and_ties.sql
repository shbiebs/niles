select k, sum(v) from q group by k order by sum(v) desc limit 3
