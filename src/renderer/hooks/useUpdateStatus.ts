import { useCallback, useEffect, useRef, useState } from 'react';

import { preferencesDesktopBridge } from '../../desktop/DesktopBridge';
import type { UpdateStatus } from '../../shared/updates';

export interface UpdateStatusController {
  readonly updateStatus: UpdateStatus | null;
  readonly checkForUpdates: () => Promise<void>;
}

export function useUpdateStatus(): UpdateStatusController {
  const [updateStatus, setUpdateStatus] = useState<UpdateStatus | null>(
    null,
  );
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    const updateBridge =
      preferencesDesktopBridge.getPreferencesUpdateBridge();

    if (updateBridge === undefined) {
      setUpdateStatus({
        phase: 'error',
        currentVersion: 'Unknown',
        message: 'Update controls are unavailable in this window.',
      });

      return () => {
        mountedRef.current = false;
      };
    }

    const unsubscribe = updateBridge.onUpdateStatusChanged(
      (nextStatus) => {
        if (mountedRef.current) {
          setUpdateStatus(nextStatus);
        }
      },
    );

    void updateBridge
      .getUpdateStatus()
      .then((nextStatus) => {
        if (mountedRef.current) {
          setUpdateStatus(nextStatus);
        }
      })
      .catch(() => {
        if (mountedRef.current) {
          setUpdateStatus({
            phase: 'error',
            currentVersion: 'Unknown',
            message: 'Update status could not be loaded.',
          });
        }
      });

    return () => {
      mountedRef.current = false;
      unsubscribe();
    };
  }, []);

  const checkForUpdates = useCallback(async (): Promise<void> => {
    const updateBridge =
      preferencesDesktopBridge.getPreferencesUpdateBridge();

    if (updateBridge === undefined) {
      setUpdateStatus({
        phase: 'error',
        currentVersion: updateStatus?.currentVersion ?? 'Unknown',
        message: 'Update controls are unavailable in this window.',
      });
      return;
    }

    try {
      const nextStatus = await updateBridge.checkForUpdates();

      if (mountedRef.current) {
        setUpdateStatus(nextStatus);
      }
    } catch {
      if (mountedRef.current) {
        setUpdateStatus({
          phase: 'error',
          currentVersion: updateStatus?.currentVersion ?? 'Unknown',
          message: 'Unable to check for updates.',
        });
      }
    }
  }, [updateStatus?.currentVersion]);

  return {
    updateStatus,
    checkForUpdates,
  };
}
