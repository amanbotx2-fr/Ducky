import { useEffect, useState } from 'react';

import { companionDesktopBridge } from '../../desktop/DesktopBridge';
import {
  createIdlePomodoroState,
  type PomodoroState,
} from '../../shared/pomodoro';

const INITIAL_STATE = createIdlePomodoroState();

export const usePomodoroState = (): PomodoroState => {
  const [state, setState] = useState<PomodoroState>(
    () =>
      companionDesktopBridge.getPomodoroBridge()?.getPomodoroState() ??
      INITIAL_STATE,
  );

  useEffect(() => {
    const bridge = companionDesktopBridge.getPomodoroBridge();

    if (bridge === undefined) {
      return;
    }

    const currentState = bridge.getPomodoroState();

    if (currentState !== null) {
      setState(currentState);
    }

    return bridge.onPomodoroStateChanged(setState);
  }, []);

  return state;
};
