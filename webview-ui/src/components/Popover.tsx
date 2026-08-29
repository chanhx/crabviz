import { createEffect, on, ParentComponent, Show } from "solid-js";

import "./Popover.css";

const Popover: ParentComponent<{
  show: () => boolean;
  hide: () => void;
}> = (props) => {
  const { show, hide } = props;
  let ref!: HTMLDivElement;
  let triggerClick = false;

  const handleClick = (e: PointerEvent) => {
    // the first click is the click on the trigger element, skip it
    if (!triggerClick) {
      triggerClick = true;
      return;
    }

    if (ref && e.target instanceof Element && !ref.contains(e.target)) {
      triggerClick = false;
      hide();
    }
  };

  createEffect(on(show, (isShow) => {
    if (isShow) {
      window.addEventListener("click", handleClick);
    } else {
      window.removeEventListener("click", handleClick);
      triggerClick = false;
    }
  }));

  return (
    <Show when={show()}>
      <div
        class="popover"
        ref={ref}
      >
        {props.children}
      </div>
    </Show>
  );
};

export default Popover;
