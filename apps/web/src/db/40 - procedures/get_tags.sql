DROP FUNCTION public.get_tags;

--
-- Name: get_tags(uuid); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE OR REPLACE FUNCTION public.get_tags(current_user_id uuid) RETURNS TABLE(tag_id uuid, tag_name text)
    LANGUAGE plpgsql
    SET search_path = public
    SECURITY INVOKER
    AS $$
BEGIN
    RETURN QUERY
    SELECT ut.id, ut.name
    FROM user_tags ut
    WHERE ut.user_id = current_user_id
    ORDER BY ut.name;
END;
$$;
