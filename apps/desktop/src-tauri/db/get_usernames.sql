SELECT username FROM cloud_ids
WHERE username IS NOT NULL
UNION
SELECT username FROM schedule_cloud_ids
WHERE username IS NOT NULL
UNION
SELECT username FROM tag_cloud_ids
WHERE username IS NOT NULL;
