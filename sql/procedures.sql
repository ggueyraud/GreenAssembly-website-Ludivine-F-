CREATE OR REPLACE PROCEDURE update_asset(p_id SMALLINT, p_order SMALLINT, p_name VARCHAR = NULL)
AS $$
DECLARE v_file_id INT;
BEGIN
	IF p_name IS NOT NULL THEN
        SELECT file_id INTO v_file_id FROM project_assets WHERE id = p_id;

        UPDATE files SET name = p_name WHERE id = v_file_id;
    END IF;

    UPDATE SET project_assets SET "order" = p_order WHERE id = p_id;
END;
$$ LANGUAGE PLPGSQL;