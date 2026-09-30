with recursive r(a, b) as (select src, dst from edges union select r.a, sum(e.dst) from r join edges e on r.b = e.src group by r.a) select * from r
