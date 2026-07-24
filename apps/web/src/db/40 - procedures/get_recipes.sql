DROP FUNCTION IF EXISTS public.get_recipes;

--
-- Name: get_recipes(uuid); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE OR REPLACE FUNCTION public.get_recipes(
    p_user_id uuid,
    p_page integer DEFAULT 1,
    p_limit integer DEFAULT 20,
    p_q text DEFAULT NULL,
    p_tags text[] DEFAULT NULL
) RETURNS TABLE(
    id uuid,
    title text,
    yield smallint,
    minutes integer,
    img_url text,
    source text,
    color text,
    ingredients jsonb,
    directions jsonb,
    last_viewed timestamp with time zone,
    tags jsonb
)
LANGUAGE plpgsql
SET search_path = public
AS $$
BEGIN
    RETURN QUERY
    SELECT
        r.id,
        r.title,
        r.yield,
        r.minutes,
        r.img_url,
        r.source,
        r.color,
        (SELECT jsonb_agg(jsonb_build_object('id', i.id, 'name', i.name, 'amount', i.amount, 'unit', i.unit))
         FROM ingredients i
         WHERE i.recipe_id = r.id) AS ingredients,
        (SELECT jsonb_agg(jsonb_build_object('id', d.id, 'content', d.content))
         FROM directions d
         WHERE d.recipe_id = r.id) AS directions,
        MAX(ru.last_viewed) AS last_viewed,
        (SELECT COALESCE(jsonb_agg(ut.name), '[]'::jsonb)
         FROM recipe_tags rt
         JOIN user_tags ut ON ut.id = rt.tag_id
         WHERE rt.recipe_id = r.id AND ut.user_id = p_user_id) AS tags
    FROM
        recipes r
    LEFT JOIN
        recipe_usage ru ON r.id = ru.recipe_id AND ru.user_id = p_user_id
    WHERE
        r.user_id = p_user_id
        AND (p_q IS NULL OR r.title ILIKE '%' || p_q || '%')
        AND (
            p_tags IS NULL OR array_length(p_tags, 1) IS NULL OR
            (
                SELECT COUNT(DISTINCT ut.name)
                FROM recipe_tags rt
                JOIN user_tags ut ON ut.id = rt.tag_id
                WHERE rt.recipe_id = r.id AND ut.user_id = p_user_id AND ut.name = ANY(p_tags)
            ) = (
                SELECT COUNT(DISTINCT t) FROM unnest(p_tags) AS t
            )
        )
    GROUP BY
        r.id, r.title, r.yield, r.minutes, r.img_url, r.source, r.color
    ORDER BY
        MAX(ru.last_viewed) DESC NULLS LAST, r.title ASC
    OFFSET (p_page - 1) * p_limit
    LIMIT p_limit;
END;
$$;


ALTER FUNCTION public.get_recipes(
    p_user_id uuid,
    p_page integer,
    p_limit integer,
    p_q text,
    p_tags text[]
) OWNER TO postgres;
