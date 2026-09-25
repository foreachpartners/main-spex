import { Route, Routes } from "react-router-dom";
import Home from "./pages/home";
import ApiTest from "./pages/api-test";
import NotFound from "./pages/not-found";

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
      <Route path="/api-test" element={<ApiTest />} />
      {/* @spec impl fe-not-found-page#unmatched-route-shows-not-found-page */}
      <Route path="*" element={<NotFound />} />
    </Routes>
  );
}
