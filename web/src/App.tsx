import { HashRouter, Routes, Route } from "react-router-dom";
import Layout from "@/components/Layout";
import MainMenu from "@/pages/MainMenu";
import HowToPlay from "@/pages/HowToPlay";
import Setup from "@/pages/Setup";
import Game from "@/pages/Game";

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<MainMenu />} />
          <Route path="/how-to-play" element={<HowToPlay />} />
          <Route path="/setup" element={<Setup />} />
          <Route path="/play" element={<Game />} />
        </Route>
      </Routes>
    </HashRouter>
  );
}
