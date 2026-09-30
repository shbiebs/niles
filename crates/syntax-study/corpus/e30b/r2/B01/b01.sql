select acct, cur, sum(amt)
from postings
where epoch <= 5
group by acct, cur
