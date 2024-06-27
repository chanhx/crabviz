import { provideVSCodeDesignSystem, vsCodeTextField, vsCodeButton } from "@vscode/webview-ui-toolkit";

provideVSCodeDesignSystem().register(vsCodeTextField(), vsCodeButton());
