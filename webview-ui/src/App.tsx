import { Component } from "solid-js";

import { Graph } from "./graph/types";

import "./styles/main.css";
import "./App.css";

import FloatingBar from "./components/FloatingBar";
import GraphViewport from "./components/GraphViewport";

const App: Component<{
  graph: Graph;
  root: string;
  focus: {
    path: string;
    line: number;
    character: number;
  } | null;
}> = (props) => {
  let focus: string | null = null;
  if (props.focus) {
    const fileId = props.focus && props.graph.files.find((f) => f.path == props.focus!.path)
      ?.id!;

    focus = `${fileId}:${props.focus!.line}_${props.focus!.character}`;
  }

  return (
    <>
      <GraphViewport graph={props.graph} root={props.root} focus={focus} />
      <FloatingBar focus={props.focus != null} />
    </>
  );
};

export default App;
