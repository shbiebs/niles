select acct, cur, txn,
       sum(amt) over (partition by acct, cur order by epoch, txn) as running
from postings
