--liquibase formatted sql

--changeset market_data:002-seed-stock-history-mock-data runAlways:true endDelimiter:/
DECLARE
    TYPE t_ticker_arr IS TABLE OF VARCHAR2(20) INDEX BY PLS_INTEGER;
    v_tickers t_ticker_arr;
    v_ticker_count PLS_INTEGER;
    c_days_per_ticker CONSTANT PLS_INTEGER := 5000; -- 20 tickers * 5,000 days = 100,000 rows

    TYPE t_date_tab IS TABLE OF DATE;
    TYPE t_ticker_tab IS TABLE OF VARCHAR2(20);
    TYPE t_num_tab IS TABLE OF NUMBER;
    v_dates   t_date_tab := t_date_tab();
    v_tks     t_ticker_tab := t_ticker_tab();
    v_opens   t_num_tab := t_num_tab();
    v_highs   t_num_tab := t_num_tab();
    v_lows    t_num_tab := t_num_tab();
    v_closes  t_num_tab := t_num_tab();
    v_volumes t_num_tab := t_num_tab();

    v_base_price NUMBER;
    v_open  NUMBER;
    v_close NUMBER;
    v_high  NUMBER;
    v_low   NUMBER;
    v_date  DATE;
    v_idx   PLS_INTEGER := 0;
    v_total PLS_INTEGER;
BEGIN
    v_tickers(1)  := 'AAPL'; v_tickers(2)  := 'MSFT'; v_tickers(3)  := 'GOOGL';
    v_tickers(4)  := 'AMZN'; v_tickers(5)  := 'TSLA'; v_tickers(6)  := 'META';
    v_tickers(7)  := 'NVDA'; v_tickers(8)  := 'NFLX'; v_tickers(9)  := 'AMD';
    v_tickers(10) := 'INTC'; v_tickers(11) := 'ORCL'; v_tickers(12) := 'IBM';
    v_tickers(13) := 'CSCO'; v_tickers(14) := 'ADBE'; v_tickers(15) := 'CRM';
    v_tickers(16) := 'PYPL'; v_tickers(17) := 'UBER'; v_tickers(18) := 'SHOP';
    v_tickers(19) := 'SQ';   v_tickers(20) := 'BABA';
    v_ticker_count := v_tickers.COUNT;
    v_total := v_ticker_count * c_days_per_ticker;

    v_dates.EXTEND(v_total);
    v_tks.EXTEND(v_total);
    v_opens.EXTEND(v_total);
    v_highs.EXTEND(v_total);
    v_lows.EXTEND(v_total);
    v_closes.EXTEND(v_total);
    v_volumes.EXTEND(v_total);

    FOR t IN 1..v_ticker_count LOOP
        v_base_price := ROUND(DBMS_RANDOM.VALUE(20, 500), 2);
        v_date := DATE '2010-01-01';

        FOR d IN 1..c_days_per_ticker LOOP
            v_open  := ROUND(v_base_price * (1 + DBMS_RANDOM.VALUE(-0.02, 0.02)), 2);
            v_close := ROUND(v_open * (1 + DBMS_RANDOM.VALUE(-0.03, 0.03)), 2);
            v_high  := ROUND(GREATEST(v_open, v_close) * (1 + DBMS_RANDOM.VALUE(0, 0.02)), 2);
            v_low   := ROUND(LEAST(v_open, v_close) * (1 - DBMS_RANDOM.VALUE(0, 0.02)), 2);

            v_idx := v_idx + 1;
            v_dates(v_idx)   := v_date;
            v_tks(v_idx)     := v_tickers(t);
            v_opens(v_idx)   := v_open;
            v_highs(v_idx)   := v_high;
            v_lows(v_idx)    := v_low;
            v_closes(v_idx)  := v_close;
            v_volumes(v_idx) := ROUND(DBMS_RANDOM.VALUE(100000, 10000000));

            v_base_price := v_close;
            v_date := v_date + 1;
        END LOOP;
    END LOOP;

    -- TRUNCATE is DDL, so it needs dynamic SQL inside a PL/SQL block.
    EXECUTE IMMEDIATE 'TRUNCATE TABLE STOCK_HISTORY';

    FORALL i IN 1..v_dates.COUNT
        INSERT INTO STOCK_HISTORY ("DATE", TICKER, "OPEN", HIGH, LOW, "CLOSE", VOLUME)
        VALUES (v_dates(i), v_tks(i), v_opens(i), v_highs(i), v_lows(i), v_closes(i), v_volumes(i));

    COMMIT;
END;
/
