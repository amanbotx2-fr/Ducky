import { handleDownloadRequest } from "../../../lib/downloads/routeHandler";

export async function GET(request: Request) {
  return handleDownloadRequest(request, "mac");
}
