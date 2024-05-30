#![cfg(feature = "alloc")]

use std::ptr::NonNull;

#[test]
fn malloc_u64_test() {
    unsafe {
        let mut p: NonNull<u64> = memsec::malloc().unwrap();
        *p.as_mut() = std::u64::MAX;
        assert_eq!(*p.as_ref(), std::u64::MAX);
        memsec::free(p);
    }
}

#[test]
fn malloc_free_test() {
    unsafe {
        let memptr: Option<NonNull<u8>> = memsec::malloc();
        assert!(memptr.is_some());
        if let Some(memptr) = memptr {
            memsec::free(memptr);
        }

        let memptr: Option<NonNull<()>> = memsec::malloc();
        assert!(memptr.is_some());
        if let Some(memptr) = memptr {
            memsec::free(memptr);
        }

        let memptr: Option<NonNull<[u8]>> = memsec::malloc_sized(1024);
        assert!(memptr.is_some());
        if let Some(memptr) = memptr {
            memsec::free(memptr);
        }

        // let memptr: Option<NonNull<[u8; std::usize::MAX - 1]>> = memsec::malloc();
        // assert!(memptr.is_none());
    }
}

#[test]
fn malloc_mprotect_1_test() {
    unsafe {
        let mut x: NonNull<[u8; 16]> = memsec::malloc().unwrap();

        memsec::memset(x.as_mut().as_mut_ptr(), 0x01, 16);
        assert!(memsec::mprotect(x, memsec::Prot::ReadOnly));
        assert!(memsec::memeq(x.as_ref().as_ptr(), [1; 16].as_ptr(), 16));
        assert!(memsec::mprotect(x, memsec::Prot::NoAccess));
        assert!(memsec::mprotect(x, memsec::Prot::ReadWrite));
        memsec::memzero(x.as_mut().as_mut_ptr(), 16);
        memsec::free(x);
    }

    unsafe {
        let mut x: NonNull<[u8; 4096]> = memsec::malloc().unwrap();
        memsec::memset(x.as_mut().as_mut_ptr(), 0x02, 96);
        memsec::free(x);
    }

    unsafe {
        let mut x: NonNull<[u8; 4100]> = memsec::malloc().unwrap();
        memsec::memset(x.as_mut().as_mut_ptr().offset(100), 0x03, 3000);
        memsec::free(x);
    }

    unsafe {
        let mut x = memsec::malloc_sized(16).unwrap();

        memsec::memset(x.as_mut().as_mut_ptr(), 0x01, 16);
        assert!(memsec::mprotect(x, memsec::Prot::ReadOnly));
        assert!(memsec::memeq(x.as_ref().as_ptr(), [1; 16].as_ptr(), 16));
        assert!(memsec::mprotect(x, memsec::Prot::NoAccess));
        assert!(memsec::mprotect(x, memsec::Prot::ReadWrite));
        memsec::memzero(x.as_mut().as_mut_ptr(), 16);
        memsec::free(x);
    }

    unsafe {
        let mut x = memsec::malloc_sized(4100).unwrap();
        memsec::memset(x.as_mut().as_mut_ptr().offset(100), 0x03, 3000);
        memsec::free(x);
    }
}

procspawn::enable_test_support!();

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Offset {
    AddOffset(usize),
    AddPages(usize),
    SubOffset(usize),
    SubPages(usize),
    Nop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
enum TestState {
    Init,
    Allocate,
    Operation,
    Free,
}

#[cfg(unix)]
fn attempt_write_in_region(
    offset: Offset,
    end_process_normally: bool,
    trigger_states: Vec<TestState>,
) {
    let (cmd_server, cmd_serv_name) = ipc_channel::ipc::IpcOneShotServer::new().unwrap();
    let (ack_server, ack_server_name) = ipc_channel::ipc::IpcOneShotServer::new().unwrap();

    //Create an external process
    let handle = procspawn::spawn(
        (offset, cmd_serv_name, ack_server_name),
        |(operation, cmd_server_name, ack_server_name)| unsafe {
            //Setup IPC channels for recieving commands
            let (tx_cmd, rx_cmd) = ipc_channel::ipc::channel().unwrap();
            let cmd_server = ipc_channel::ipc::IpcSender::connect(cmd_server_name).unwrap();
            cmd_server.send(tx_cmd).unwrap();

            //Setup IPC channels for acknowledging state completion
            let (tx_ack, rx_ack) = ipc_channel::ipc::channel().unwrap();
            let ack_server = ipc_channel::ipc::IpcSender::connect(ack_server_name).unwrap();
            ack_server.send(rx_ack).unwrap();

            let mut page_size = None;
            let mut p: Option<NonNull<u64>> = None;

            loop {
                let rval = rx_cmd.recv().unwrap();

                match rval {
                    TestState::Init => {
                        page_size = Some(libc::sysconf(libc::_SC_PAGESIZE) as usize);
                        tx_ack.send(rval).unwrap();
                    }

                    TestState::Allocate => {
                        p = Some(memsec::malloc().unwrap());
                        tx_ack.send(rval).unwrap();
                    }

                    TestState::Operation => {
                        let ptr = p.unwrap().as_ptr() as *mut u8;

                        match operation {
                            Offset::AddOffset(offset) => {
                                let page_after = ptr.add(offset);
                                *page_after = 0x01;
                            }
                            Offset::SubOffset(offset) => {
                                let page_before = ptr.sub(offset);
                                *page_before = 0x01;
                            }
                            Offset::Nop => {
                                *ptr = 0x01;
                            }

                            Offset::AddPages(pages) => {
                                let page_after = ptr.add(pages * page_size.unwrap());
                                *page_after = 0x01;
                            }
                            Offset::SubPages(pages) => {
                                let page_before = ptr.sub(pages * page_size.unwrap());
                                *page_before = 0x01;
                            }
                        }
                        tx_ack.send(rval).unwrap();
                    }
                    TestState::Free => {
                        memsec::free(p.unwrap());
                        tx_ack.send(rval).unwrap();
                        return;
                    }
                }
            }
        },
    );

    let (_, tx): (_, ipc_channel::ipc::IpcSender<TestState>) = cmd_server.accept().unwrap();

    let (_, rx): (_, ipc_channel::ipc::IpcReceiver<TestState>) = ack_server.accept().unwrap();

    for &state in trigger_states[..trigger_states.len() - 1].iter() {
        tx.send(state).unwrap();
        assert_eq!(state, rx.try_recv_timeout(Duration::from_secs(1)).unwrap());
    }

    let state = trigger_states[trigger_states.len() - 1];
    tx.send(state).unwrap();

    //If the process is expected to end normally, then the last state should be received
    if end_process_normally {
        assert_eq!(state, rx.try_recv_timeout(Duration::from_secs(1)).unwrap());
    }

    let r = handle.join();

    assert!(r.is_ok() == end_process_normally);
}

#[cfg(unix)]
#[test]
fn malloc_probe_outside_normal() {
    attempt_write_in_region(
        Offset::Nop,
        true,
        vec![
            TestState::Init,
            TestState::Allocate,
            TestState::Operation,
            TestState::Free,
        ],
    );
}

#[cfg(unix)]
#[test]
fn malloc_probe_outside_limits_canary() {
    //Canary changes crash the process
    attempt_write_in_region(
        Offset::SubOffset(1),
        false,
        vec![
            TestState::Init,
            TestState::Allocate,
            TestState::Operation,
            TestState::Free,
        ],
    );
}

#[cfg(unix)]
#[test]
fn malloc_probe_outside_limits_page_above() {
    attempt_write_in_region(
        Offset::SubPages(1),
        false,
        vec![TestState::Init, TestState::Allocate, TestState::Operation],
    );
}

#[cfg(unix)]
#[test]
fn malloc_probe_outside_limits_two_pages_above() {
    attempt_write_in_region(
        Offset::SubPages(2),
        false,
        vec![TestState::Init, TestState::Allocate, TestState::Operation],
    );
}

#[cfg(unix)]
#[test]
fn malloc_probe_outside_limits_page_after_above() {
    attempt_write_in_region(
        Offset::AddPages(1),
        false,
        vec![TestState::Init, TestState::Allocate, TestState::Operation],
    );
}
