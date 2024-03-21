import { render } from "solid-js/web";
import { lazy } from "solid-js";
import { Router, Route } from "@solidjs/router";

import App from "./App";
import "./index.scss";

const Library = lazy(() => import("./components/library"));

render(
  () => (
    <Router root={App}>
      <Route path="/test" component={Library} />
    </Router>
  ),
  document.getElementById("root")!
);
