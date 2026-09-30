select acct, cur, txn,
       sum(coalesce((amt_usd).minor, (amt_eur).minor, (amt_jpy).minor))
           over (partition by acct, cur order by epoch, txn) as running
from postings
