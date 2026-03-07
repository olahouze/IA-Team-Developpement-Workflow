"""Tests unitaires pour launcher/run.py"""
import os
import sys
import socket
from unittest.mock import patch, MagicMock

import pytest

# Add launcher directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
import run


class TestIsDockerRunning:
    """Tests for is_docker_running()"""

    @patch('run.subprocess.run')
    def test_returns_true_when_container_running(self, mock_run):
        mock_run.return_value = MagicMock(stdout='abc123\n')
        assert run.is_docker_running('windmill-db') is True
        mock_run.assert_called_once()

    @patch('run.subprocess.run')
    def test_returns_false_when_container_not_running(self, mock_run):
        mock_run.return_value = MagicMock(stdout='')
        assert run.is_docker_running('windmill-db') is False

    @patch('run.subprocess.run', side_effect=FileNotFoundError)
    def test_returns_false_when_docker_not_installed(self, mock_run):
        assert run.is_docker_running('windmill-db') is False


class TestCheckPorts:
    """Tests for check_ports()"""

    @patch('socket.socket')
    def test_passes_when_ports_free(self, mock_socket_class):
        mock_sock = MagicMock()
        mock_sock.__enter__ = MagicMock(return_value=mock_sock)
        mock_sock.__exit__ = MagicMock(return_value=False)
        # connect_ex returning nonzero means port is free
        mock_sock.connect_ex.return_value = 1
        mock_socket_class.return_value = mock_sock

        # Should not raise SystemExit
        run.check_ports()

    @patch('socket.socket')
    def test_exits_when_port_in_use(self, mock_socket_class):
        mock_sock = MagicMock()
        mock_sock.__enter__ = MagicMock(return_value=mock_sock)
        mock_sock.__exit__ = MagicMock(return_value=False)
        # connect_ex returning 0 means port IS in use
        mock_sock.connect_ex.return_value = 0
        mock_socket_class.return_value = mock_sock

        with pytest.raises(SystemExit):
            run.check_ports()


class TestDownloadSkipLogic:
    """Tests for download functions skip logic"""

    @patch('os.path.exists', return_value=True)
    @patch('os.listdir', return_value=['postgres.exe', 'initdb.exe'])
    def test_postgres_skip_if_already_present(self, mock_listdir, mock_exists):
        # Should return without downloading
        run.download_and_extract_postgres()
        # No requests should have been made

    @patch('os.path.exists', return_value=True)
    def test_windmill_skip_if_already_present(self, mock_exists):
        # When the binary already exists, skip download
        run.download_and_install_windmill()


class TestPortConstants:
    """Tests for port configuration"""

    def test_ports_are_distinct(self):
        ports = [run.TAURI_PORT, run.WINDMILL_PORT, run.PGSQL_PORT]
        assert len(ports) == len(set(ports)), "Ports must be distinct"

    def test_ports_are_valid(self):
        for port in [run.TAURI_PORT, run.WINDMILL_PORT, run.PGSQL_PORT]:
            assert 1 <= port <= 65535, f"Port {port} is out of valid range"

    def test_expected_default_values(self):
        assert run.TAURI_PORT == 1420
        assert run.WINDMILL_PORT == 8000
        assert run.PGSQL_PORT == 5432
