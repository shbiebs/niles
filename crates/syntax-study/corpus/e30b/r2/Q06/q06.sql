select acct, count(distinct cur)
from postings
group by acct
