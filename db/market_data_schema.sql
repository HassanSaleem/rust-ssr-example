-- 1. Ensure you are in the correct application database
ALTER SESSION SET CONTAINER = FREEPDB1;

-- 2. Create the user inside this database container
CREATE USER market_data IDENTIFIED BY "your_secure_password_123";

-- 3. Grant the required permissions
GRANT CREATE SESSION, 
      CREATE TABLE, 
      CREATE VIEW, 
      CREATE SEQUENCE, 
      CREATE PROCEDURE, 
      CREATE TRIGGER,
      CREATE SYNONYM TO market_data;

-- 4. Grant storage quota space
ALTER USER market_data QUOTA UNLIMITED ON USERS;