CREATE OR REPLACE FUNCTION insert_website RETURNS TRIGGER AS $$
BEGIN
    IF (SELECT COUNT(*) FROM website > 1) THEN
        RAISE EXCEPTION 'Only one row can be created for the website table';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;
CREATE trigger_insert_website BEFORE INSERT ON website ROW EXECUTE PROCEDURE insert_website();

CREATE OR REPLACE FUNCTION insert_user RETURNS TRIGGER AS $$
BEGIN
    IF (SELECT COUNT(*) FROM user > 1) THEN
        RAISE EXCEPTION 'Only one row can be created for the user table';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;
CREATE trigger_insert_user BEFORE INSERT ON user ROW EXECUTE PROCEDURE insert_user();