WITH affected(name) AS (
    SELECT t.name FROM sqlite_schema AS t
    WHERE t.type = 'table' AND t.sql NOT LIKE 'CREATE VIRTUAL TABLE%'
      AND (t.name IN ('saved_lists', 'parties', 'encounters') OR EXISTS (
        SELECT 1 FROM pragma_foreign_key_list(t.name) AS fk
        WHERE fk."table" IN ('saved_lists', 'parties', 'encounters')
      ))
)
SELECT type, name, tbl_name, sql FROM sqlite_schema
WHERE tbl_name IN (SELECT name FROM affected) AND sql IS NOT NULL
ORDER BY type, name;
