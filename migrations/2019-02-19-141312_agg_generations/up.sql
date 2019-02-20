create materialized view if not exists public.agg_generations as
    ( select kb.*, COALESCE(jsonb_agg( mb.*) FILTER (WHERE mb.id IS NOT NULL), '[]') 
    as micro_blocks from public.key_blocks kb left join 
    ( select m.*, COALESCE(jsonb_agg( tx.*) FILTER (WHERE tx.id IS NOT NULL), '[]') 
    as transactions from public.micro_blocks m left join public.transactions tx on m.id = tx.micro_block_id group by m.id ) 
    mb on kb.id = mb.key_block_id where kb.height >=0 group by kb.id ) with no data;

create index agg_generations_height_idx ON public.agg_generations (height);
refresh materialized view public.agg_generations;