import React, { useState, useEffect, useCallback } from 'react';
import { Download, RefreshCw, Sparkles, X, ArrowDownToLine, Loader2 } from 'lucide-react';
import { AppUpdateInfo } from '../types';

interface AppUpdateBannerProps {
  /** Pass null to hide, or the update info to display */
  updateInfo: AppUpdateInfo | null;
  onDismiss: () => void;
}

export const AppUpdateBanner: React.FC<AppUpdateBannerProps> = ({ updateInfo, onDismiss }) => {
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadStage, setDownloadStage] = useState<string>('');
  const [downloadPercent, setDownloadPercent] = useState(0);
  const [error, setError] = useState<string | null>(null);

  // Listen for update progress events from Rust
  useEffect(() => {
    if (!window.riftAPI?.onUpdateProgress) return;
    const cleanup = window.riftAPI.onUpdateProgress((data) => {
      setDownloadStage(data.stage);
      setDownloadPercent(data.percent);
    });
    return cleanup;
  }, []);

  const handleUpdate = useCallback(async () => {
    if (!updateInfo?.downloadUrl || !window.riftAPI?.downloadAppUpdate) return;
    setIsDownloading(true);
    setError(null);
    try {
      await window.riftAPI.downloadAppUpdate(updateInfo.downloadUrl);
    } catch (err: any) {
      setError(err?.message || 'Download failed');
      setIsDownloading(false);
    }
  }, [updateInfo]);

  if (!updateInfo || !updateInfo.hasUpdate) return null;

  const sizeInMB = updateInfo.assetSize > 0
    ? (updateInfo.assetSize / (1024 * 1024)).toFixed(1)
    : null;

  return (
    <div className="app-update-banner">
      <div className="update-banner-glow" />
      <div className="update-banner-content">
        <div className="update-banner-icon">
          <Sparkles size={16} />
        </div>
        <div className="update-banner-text">
          <span className="update-banner-title">
            RiftCore v{updateInfo.latestVersion} is available!
          </span>
          <span className="update-banner-subtitle">
            You're on v{updateInfo.currentVersion}
            {sizeInMB && <> · {sizeInMB} MB</>}
          </span>
        </div>

        <div className="update-banner-actions">
          {error && (
            <span className="update-banner-error">{error}</span>
          )}
          {isDownloading ? (
            <div className="update-banner-progress">
              <Loader2 size={14} className="spin" />
              <span>
                {downloadStage === 'installing'
                  ? 'Launching installer...'
                  : 'Downloading...'}
              </span>
            </div>
          ) : (
            <button
              className="update-banner-btn update-btn-primary"
              onClick={handleUpdate}
              disabled={!updateInfo.downloadUrl}
              title="Download and install the latest version"
            >
              <ArrowDownToLine size={14} />
              Update Now
            </button>
          )}
          <button
            className="update-banner-btn update-btn-dismiss"
            onClick={onDismiss}
            title="Dismiss this notification"
          >
            <X size={14} />
          </button>
        </div>
      </div>
    </div>
  );
};
