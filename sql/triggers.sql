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

CREATE OR REPLACE FUNCTION delete_project_category() RETURNS TRIGGER AS $$
DECLARE i SMALLINT;
DECLARE row RECORD;
BEGIN
    i := 1;

    FOR row IN SELECT id, "order" FROM project_categories
        LOOP
            RAISE LOG 'Attempt to update % from % to "order" value %', row.id, row.order, i;

            UPDATE project_categories SET "order" = i WHERE id = row.id;
            i := i + 1;
        END LOOP;

    RETURN OLD;
END;
$$ LANGUAGE PLPGSQL;
DROP TRIGGER IF EXISTS trigger_delete_project_category ON project_categories;
CREATE TRIGGER trigger_delete_project_category AFTER DELETE ON project_categories FOR EACH ROW EXECUTE PROCEDURE delete_project_category();

CREATE OR REPLACE FUNCTION update_project_category() RETURNS TRIGGER AS $$
DECLARE
    row RECORD;
    rows_ids SMALLINT[];
    row_id SMALLINT;
    space SMALLINT;
BEGIN
    IF NEW.order IS NOT NULL THEN
        space := 1;

        IF NEW.order > OLD.order THEN
            SELECT ARRAY(SELECT id FROM project_categories WHERE "order" <= NEW.order AND id != NEW.id ORDER BY "order" DESC) INTO rows_ids;

            FOREACH row_id IN ARRAY rows_ids
            LOOP
                UPDATE project_categories SET "order" = NEW.order - space WHERE id = row_id;

                space := space + 1;
            END LOOP;
        ELSE
            SELECT ARRAY(SELECT id FROM project_categories WHERE "order" >= NEW.order AND id != NEW.id ORDER BY "order" ASC) INTO rows_ids;

            FOREACH row_id IN ARRAY rows_ids
            LOOP
                UPDATE project_categories SET "order" = NEW.order + space WHERE id = row_id;

                space := space + 1;
            END LOOP;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;
DROP TRIGGER IF EXISTS trigger_update_project_category ON project_categories;
CREATE TRIGGER trigger_update_project_category BEFORE UPDATE ON project_categories FOR EACH ROW WHEN (pg_trigger_depth() < 1) EXECUTE PROCEDURE update_project_category();

CREATE OR REPLACE FUNCTION update_blog_category() RETURNS TRIGGER AS $$
DECLARE
    row RECORD;
    rows_ids SMALLINT[];
    row_id SMALLINT;
    space SMALLINT;
BEGIN
    IF NEW.order IS NOT NULL THEN
        space := 1;

        IF NEW.order > OLD.order THEN
            SELECT ARRAY(SELECT id FROM blog_categories WHERE "order" <= NEW.order AND id != NEW.id ORDER BY "order" DESC) INTO rows_ids;

            FOREACH row_id IN ARRAY rows_ids
            LOOP
                UPDATE blog_categories SET "order" = NEW.order - space WHERE id = row_id;

                space := space + 1;
            END LOOP;
        ELSE
            SELECT ARRAY(SELECT id FROM blog_categories WHERE "order" >= NEW.order AND id != NEW.id ORDER BY "order" ASC) INTO rows_ids;

            FOREACH row_id IN ARRAY rows_ids
            LOOP
                UPDATE blog_categories SET "order" = NEW.order + space WHERE id = row_id;

                space := space + 1;
            END LOOP;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE PLPGSQL;
DROP TRIGGER IF EXISTS trigger_update_blog_category ON blog_categories;
CREATE TRIGGER trigger_update_blog_category BEFORE UPDATE ON blog_categories FOR EACH ROW WHEN (pg_trigger_depth() < 1) EXECUTE PROCEDURE update_blog_category();

CREATE OR REPLACE FUNCTION delete_blog_category() RETURNS TRIGGER AS $$
DECLARE i SMALLINT;
DECLARE row RECORD;
BEGIN
    i := 1;

    FOR row IN SELECT id, "order" FROM blog_categories
        LOOP
            RAISE LOG 'Attempt to update % from % to "order" value %', row.id, row.order, i;

            UPDATE blog_categories SET "order" = i WHERE id = row.id;
            i := i + 1;
        END LOOP;

    RETURN OLD;
END;
$$ LANGUAGE PLPGSQL;
DROP TRIGGER IF EXISTS trigger_delete_blog_category ON blog_categories;
CREATE TRIGGER trigger_delete_blog_category AFTER DELETE ON blog_categories FOR EACH ROW EXECUTE PROCEDURE delete_blog_category();