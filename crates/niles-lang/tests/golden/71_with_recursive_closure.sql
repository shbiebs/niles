with recursive r(a, b) as (select src, dst from edges union select r.a, e.dst from r join edges e on r.b = e.src) select * from r
