import { LicenseManager } from "ag-grid-enterprise";

// Call once before any AG Grid component mounts. Key lives in .env.local (gitignored).
const licenseKey = process.env.NEXT_PUBLIC_AG_GRID_LICENSE_KEY;
if (licenseKey) {
  LicenseManager.setLicenseKey(licenseKey);
}
