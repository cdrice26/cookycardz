SELECT recipe_id AS value FROM cloud_ids
WHERE cloud_recipe_id = $1 AND username = $2;
