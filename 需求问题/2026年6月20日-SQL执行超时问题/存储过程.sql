DELIMITER $$

CREATE PROCEDURE sp_copy_trade_order_history(
    IN start_num INT,          -- 起始序号，默认 1
    IN end_num INT,            -- 结束序号，默认 10000
    IN base_table VARCHAR(64)  -- 原表名，默认 'tb_trade_order_history'
)
BEGIN
    DECLARE i INT DEFAULT start_num;
    DECLARE new_table_name VARCHAR(128);

    -- 设置默认值
    IF start_num IS NULL THEN SET start_num = 1; END IF;
    IF end_num IS NULL THEN SET end_num = 10000; END IF;
    IF base_table IS NULL THEN SET base_table = 'tb_trade_order_history'; END IF;

    WHILE i <= end_num DO
        SET new_table_name = CONCAT(base_table, '_', i);
        -- 关键修改：将动态 SQL 赋值给用户变量 @sql_stmt
        SET @sql_stmt = CONCAT('CREATE TABLE IF NOT EXISTS `', new_table_name, '` LIKE `', base_table, '`');

        PREPARE stmt FROM @sql_stmt;   -- 此处使用用户变量
        EXECUTE stmt;
        DEALLOCATE PREPARE stmt;

        IF i % 1000 = 0 THEN
            SELECT CONCAT('已创建 ', i, ' 张表') AS progress;
        END IF;

        SET i = i + 1;
    END WHILE;

    SELECT CONCAT('全部 ', end_num - start_num + 1, ' 张表创建完成！') AS result;
END$$

DELIMITER ;