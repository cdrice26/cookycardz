-- $1 = query
-- $2 = tags
-- $3 = page
-- $4 = limit

WITH RECURSIVE split(value, rest) AS (
    SELECT '', $2 || ','
    UNION ALL
    SELECT
        trim(substr(rest, 1, instr(rest, ',') - 1)),
        substr(rest, instr(rest, ',') + 1)
    FROM split
    WHERE rest <> ''
)
SELECT
    r.id,
    r.title,
    r.yield,
    r.minutes,
    r.img_url,
    r.source,
    r.color,
    MAX(ru.last_viewed) AS last_viewed
FROM
    recipes r
LEFT JOIN
    recipe_usage ru ON r.id = ru.recipe_id
WHERE ($1 IS NULL OR r.title LIKE '%' || $1 || '%')
    AND (
        $2 IS NULL OR $2 = '' OR
        (
            SELECT COUNT(DISTINCT ut.name)
            FROM recipe_tags rt
            JOIN user_tags ut ON ut.id = rt.tag_id
            WHERE rt.recipe_id = r.id
                AND ut.name IN (SELECT value FROM split WHERE value <> '')
        ) = (
            SELECT COUNT(DISTINCT value) FROM split where value <> ''
        )
    )
GROUP BY
    r.id, r.title, r.yield, r.minutes, r.img_url, r.source, r.color
ORDER BY
    MAX(ru.last_viewed) DESC, r.title ASC
LIMIT $4 OFFSET ($3 - 1) * $4;
