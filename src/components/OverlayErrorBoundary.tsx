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

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('[RiftCore Overlay] Runtime error caught:', error, errorInfo);
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
            style={{
              pointerEvents: 'auto',
              padding: '12px 18px',
              borderRadius: '10px',
              background: 'rgba(10, 16, 28, 0.95)',
              border: '1px solid rgba(255, 42, 95, 0.4)',
              boxShadow: '0 8px 32px rgba(0, 0, 0, 0.8)',
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              color: '#fff',
              fontSize: '12px',
            }}
          >
            <AlertTriangle size={18} color="#ff2a5f" />
            <div>
              <div style={{ fontWeight: 800, color: '#f8fafc' }}>Overlay Telemetry Syncing...</div>
              <div style={{ fontSize: '10px', color: '#94a3b8' }}>Recovering active game state</div>
            </div>
            <button
              onClick={() => {
                this.setState({ hasError: false });
                if (this.props.onExpand) this.props.onExpand();
              }}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '5px',
                padding: '4px 10px',
                borderRadius: '6px',
                background: 'rgba(0, 210, 255, 0.2)',
                border: '1px solid var(--blue-primary)',
                color: 'var(--blue-primary)',
                fontSize: '11px',
                cursor: 'pointer',
                fontWeight: 700,
              }}
            >
              <RotateCcw size={12} />
              <span>Retry</span>
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
