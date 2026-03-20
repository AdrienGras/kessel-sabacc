import { HashRouter, Routes, Route } from "react-router-dom";
import Layout from "@/components/Layout";
import Intro from "@/pages/Intro";
import MainMenu from "@/pages/MainMenu";
import HowToPlay from "@/pages/HowToPlay";
import Setup from "@/pages/Setup";
import Game from "@/pages/Game";
import Credits from "@/pages/Credits";

export default function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<Intro />} />
        <Route element={<Layout />}>
          <Route path="/menu" element={<MainMenu />} />
          <Route path="/how-to-play" element={<HowToPlay />} />
          <Route path="/setup" element={<Setup />} />
          <Route path="/play" element={<Game />} />
          <Route path="/credits" element={<Credits />} />
        </Route>
      </Routes>
    </HashRouter>
  );
}
