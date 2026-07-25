import { handleDownloadRequest } from "../../../lib/downloads/routeHandler";

export async function GET() {
  return handleDownloadRequest("mac");
}
