import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RotateCcw } from 'lucide-react';

interface Props {
  children: ReactNode;
  onExpand?: () => void;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class OverlayErrorBoundary extends Component<Props, State> {
  public state: State = {
    hasError: false,
  };

  private recoveryTimer: any = null;

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('[RiftCore Overlay] Runtime error caught:', error, errorInfo);
    // Automatically attempt recovery after 1.5 seconds so a temporary loading glitch never freezes the overlay
    if (this.recoveryTimer) clearTimeout(this.recoveryTimer);
    this.recoveryTimer = setTimeout(() => {
      this.setState({ hasError: false, error: undefined });
    }, 1500);
  }

  public componentDidUpdate(prevProps: Props) {
    if (this.state.hasError && this.props.children !== prevProps.children) {
      if (this.recoveryTimer) clearTimeout(this.recoveryTimer);
      this.setState({ hasError: false, error: undefined });
    }
  }

  public componentWillUnmount() {
    if (this.recoveryTimer) clearTimeout(this.recoveryTimer);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div
          style={{
            width: '100vw',
            height: '100vh',
            background: 'transparent',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            pointerEvents: 'none',
          }}
        >
          <div
            data-rift-interactive="true"
            style={{
              pointerEvents: 'auto',
              padding: '10px 16px',
              borderRadius: '10px',
              background: 'rgba(10, 16, 28, 0.95)',
              border: '1px solid rgba(0, 210, 255, 0.4)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.8), 0 0 16px rgba(0, 210, 255, 0.2)',
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              color: '#fff',
              fontSize: '12px',
            }}
          >
            <AlertTriangle size={16} color="#38bdf8" />
            <div>
              <div style={{ fontWeight: 800, color: '#f8fafc', fontSize: '11.5px' }}>Overlay Telemetry Syncing...</div>
              <div style={{ fontSize: '9.5px', color: '#94a3b8' }}>Auto-recovering live match data</div>
            </div>
            <button
              onClick={() => {
                if (this.recoveryTimer) clearTimeout(this.recoveryTimer);
                this.setState({ hasError: false, error: undefined });
                if (this.props.onExpand) this.props.onExpand();
              }}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
                padding: '3px 9px',
                borderRadius: '6px',
                background: 'rgba(0, 210, 255, 0.2)',
                border: '1px solid var(--blue-primary)',
                color: 'var(--blue-primary)',
                fontSize: '10.5px',
                cursor: 'pointer',
                fontWeight: 800,
              }}
            >
              <RotateCcw size={11} />
              <span>Retry</span>
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
