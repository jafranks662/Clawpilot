import "./globals.css";
import { ConvexClientProvider } from "@/components/convex-client";

export const metadata = {
  title: "Mission Control",
  description: "Task, content, memory, team, and calendar operations"
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>
        <ConvexClientProvider>{children}</ConvexClientProvider>
      </body>
    </html>
  );
}
