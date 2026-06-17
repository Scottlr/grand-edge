import { AppShell } from "./layout/AppShell";
import { GlossaryProvider } from "./components/learn/GlossaryProvider";

export default function App() {
  return (
    <GlossaryProvider>
      <AppShell />
    </GlossaryProvider>
  );
}
