import {
  batch,
  createMemo,
  createSignal,
  Component,
  Show,
  Suspense,
} from "solid-js";
import { VList } from "virtua/solid";

import { useAppContext, ScaleOption, ExportOption } from "../context.tsx";
import { kindNum2Name } from "../lsp";

import Popover from "./Popover.tsx";
import Spinner from "./Spinner";
import Switch from "./Switch.tsx";

import "./FloatingBar.css";

import svgCollapse from "../assets/collapse.svg?raw";
import svgExpand from "../assets/expand.svg?raw";
import svgSearch from "../assets/search.svg?raw";
import svgGoto from "../assets/goto.svg?raw";

interface IOption {
  id: string;
  label: string;
  kind: string | null;
  detail: string | null;
}

const FloatingBar: Component<{ focus: boolean }> = (props) => {
  const [{ }, { setCollapse, setScaleOpt, setExportOpt }] = useAppContext();

  const [showSaveOptions, setShowSaveOptions] = createSignal(false);

  const [{ items, selectedElem }, { setSelectedElem }] = useAppContext();

  const [query, setQuery] = createSignal<string | undefined>(undefined);

  const options = createMemo(() => {
    const _items = items();
    if (!_items) {
      return undefined;
    }

    const files = Array.from(_items.files.values(), file2option);
    const symbols = _items.symbols.map(symbol2option);

    return files.concat(symbols).sort((a, b) => a.id.localeCompare(b.id));
  });
  const filtered = createMemo(() => {
    const q = query()?.toLowerCase();
    if (!q) {
      return options();
    }
    return options()?.filter((o) => o.label.toLowerCase().includes(q));
  });

  const jump = () => {
    const elem = selectedElem();
    if (!elem) {
      return;
    }
    const classes = elem.classList;

    let file_id: string;
    let ln = 0,
      col = 0;
    if (classes.contains("node")) {
      file_id = elem.id;
    } else {
      let s = elem.id.split(":");
      file_id = s[0];
      [ln, col] = s[1].split("_").map((s) => parseInt(s));
    }

    window.postMessage({
      command: "go to definition",
      path: items()!.files.get(file_id)!.dataset.path,
      ln,
      col,
    });
  };

  const selectedSymbol = (): SVGElement | null => {
    const elem = selectedElem();
    const classes = elem?.classList;

    if (classes?.contains("cell") || classes?.contains("node")) {
      return elem;
    } else {
      return null;
    }
  };

  return (
    <div class="floating-bar">
      <div class="search">
        <button
          class="button"
          onClick={() => setQuery("")}
          innerHTML={svgSearch}
        ></button>
        <Show when={selectedSymbol()} keyed>
          {
            (symbol) =>
              <>
                <Option option={elem2option(symbol)} />
                <button
                  class="button"
                  onClick={jump}
                  innerHTML={svgGoto}
                ></button>
              </>
          }
        </Show>

        <Popover
          show={() => query() !== undefined}
          hide={() => setQuery(undefined)}
        >
          <input
            type="text"
            value={query()}
            onInput={(e) => setQuery(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                setQuery(undefined);
              }
            }}
            placeholder="Search file or symbol..."
          ></input>

          <div class="option-list">
            <Suspense fallback={<Spinner />}>
              <Show
                when={filtered()?.length}
                fallback={<div class="prompt">No result</div>}
              >
                <VList
                  data={filtered()!}
                  style={{
                    height: `calc(${filtered()!.length} * var(--option-height))`,
                  }}
                >
                  {(option) => (
                    <Option
                      option={option}
                      onClick={(o) => {
                        batch(() => {
                          setSelectedElem(
                            document.querySelector<SVGElement>(`[id="${o.id}"]`)!
                          );
                          setQuery(undefined);
                        });
                      }}
                    />
                  )}
                </VList>
              </Show>
            </Suspense>
          </div>
        </Popover>
      </div>

      <Show when={!props.focus}>
        <Switch
          onChange={(isChecked) => {
            setCollapse(!isChecked);
          }}
          title="switch to expand files"
          titleChecked="switch to collapse files"
          icon={svgCollapse}
          iconChecked={svgExpand}
        />
      </Show>

      <button class="button" onClick={() => setScaleOpt(ScaleOption.Reset)}>
        Reset
      </button>

      <button class="button save-btn" onClick={() => setShowSaveOptions(true)}>
        Save
        <Popover show={() => showSaveOptions()} hide={() => setShowSaveOptions(false)}>
          <div class="option-list">
            <label>as ...</label>
            <div class="option"
              onClick={() => {
                setExportOpt(ExportOption.Html);
                setShowSaveOptions(false);
              }}
            >
              HTML
            </div>
            <div class="option"
              onClick={() => {
                setExportOpt(ExportOption.Svg);
                setShowSaveOptions(false);
              }}
            >
              SVG
            </div>
          </div>
        </Popover>
      </button>
    </div>
  );
};

const Option: Component<{
  option: IOption;
  onClick?: (option: IOption) => void;
}> = (props) => {
  const option = props.option,
    onClick = props.onClick,
    kind = option.kind ?? "";

  return (
    <div
      class="option"
      onMouseDown={(e) => e.preventDefault()}
      onClick={() => onClick && onClick(option)}
    >
      <span class={`kind ${kind}`}>{kind}</span>
      <div class="label">{option.label}</div>
    </div>
  );
};

const elem2option = (e: SVGElement): IOption => {
  if (e.dataset.path) {
    return file2option(e);
  } else {
    return symbol2option(e);
  }
};

const file2option = (e: SVGElement): IOption => {
  return {
    id: e.id,
    label: e.querySelector(".title")!.firstElementChild!.textContent!.trim(),
    kind: null,
    detail: e.dataset.path!,
  };
};

const symbol2option = (e: SVGElement): IOption => {
  return {
    id: e.id,
    label: e.querySelector(":scope > text:last-of-type")!.textContent!,
    kind: kindNum2Name(e.dataset.kind!),
    detail: null,
  };
};

export default FloatingBar;
