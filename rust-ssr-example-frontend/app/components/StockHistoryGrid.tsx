"use client";

import { useMemo } from "react";
import { AgGridReact } from "ag-grid-react";
import {
  AllCommunityModule,
  ColDef,
  IDatasource,
  IGetRowsParams,
  ModuleRegistry,
  themeQuartz,
} from "ag-grid-community";
import { AllEnterpriseModule } from "ag-grid-enterprise";

import "@/lib/ag-grid-license";
import { getRows } from "@/lib/__generated__/domains-stock-history-routes/domains-stock-history-routes";
import type { StockHistoryRow } from "@/lib/__generated__/models";

ModuleRegistry.registerModules([AllCommunityModule, AllEnterpriseModule]);

const PAGE_SIZE = 100;

const columnDefs: ColDef<StockHistoryRow>[] = [
  { field: "date", headerName: "Date", filter: "agDateColumnFilter", sort: "desc" },
  { field: "ticker", headerName: "Ticker", filter: "agTextColumnFilter" },
  { field: "open", headerName: "Open", filter: "agNumberColumnFilter" },
  { field: "high", headerName: "High", filter: "agNumberColumnFilter" },
  { field: "low", headerName: "Low", filter: "agNumberColumnFilter" },
  { field: "close", headerName: "Close", filter: "agNumberColumnFilter" },
  { field: "volume", headerName: "Volume", filter: "agNumberColumnFilter" },
];

export default function StockHistoryGrid() {
  // Bridges AG Grid's Infinite Row Model datasource to the backend's getRows endpoint.
  const datasource: IDatasource = useMemo(
    () => ({
      getRows: (params: IGetRowsParams) => {
        getRows({
          startRow: params.startRow,
          endRow: params.endRow,
          sortModel: params.sortModel,
          filterModel: params.filterModel,
        })
          .then((response) => {
            if (response.status === 200) {
              params.successCallback(response.data.rowData, response.data.rowCount);
            } else {
              params.failCallback();
            }
          })
          .catch(() => params.failCallback());
      },
    }),
    [],
  );

  return (
    <div style={{ height: 600, width: "100%" }}>
      <AgGridReact<StockHistoryRow>
        theme={themeQuartz}
        columnDefs={columnDefs}
        rowModelType="infinite"
        datasource={datasource}
        cacheBlockSize={PAGE_SIZE}
        maxBlocksInCache={10}
      />
    </div>
  );
}
