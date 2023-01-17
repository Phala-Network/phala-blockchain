// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

use crate::mutex::MovableMutex;
use crate::sys::condvar as imp;
use crate::sys::mutex as mutex_imp;
use core::time::Duration;
use sgx_types::error::errno::ETIMEDOUT;

mod check;

type CondvarCheck = <mutex_imp::MovableMutex as check::CondvarCheck>::Check;

/// An SGX-based condition variable.
pub struct Condvar {
    inner: imp::MovableCondvar,
    check: CondvarCheck,
}

impl Condvar {
    /// Creates a new condition variable for use.
    pub fn new() -> Condvar {
        let c = imp::MovableCondvar::from(imp::Condvar::new());
        Condvar {
            inner: c,
            check: CondvarCheck::new(),
        }
    }

    /// Signals one waiter on this condition variable to wake up.
    #[inline]
    pub fn notify_one(&self) {
        let r = unsafe { self.inner.notify_one() };
        debug_assert_eq!(r, Ok(()));
    }

    /// Awakens all current waiters on this condition variable.
    #[inline]
    pub fn notify_all(&self) {
        let r = unsafe { self.inner.notify_all() };
        debug_assert_eq!(r, Ok(()));
    }

    /// Waits for a signal on the specified mutex.
    ///
    /// Behavior is undefined if the mutex is not locked by the current thread.
    ///
    /// May panic if used with more than one mutex.
    #[inline]
    pub unsafe fn wait(&self, mutex: &MovableMutex) {
        self.check.verify(mutex);
        let r = self.inner.wait(mutex.as_ref());
        debug_assert_eq!(r, Ok(()));
    }

    /// Waits for a signal on the specified mutex with a timeout duration
    /// specified by `dur` (a relative time into the future).
    ///
    /// Behavior is undefined if the mutex is not locked by the current thread.
    ///
    /// May panic if used with more than one mutex.
    #[inline]
    pub unsafe fn wait_timeout(&self, mutex: &MovableMutex, dur: Duration) -> bool {
        self.check.verify(mutex);
        let r = self.inner.wait_timeout(mutex.as_ref(), dur);
        debug_assert!(r == Err(ETIMEDOUT) || r == Ok(()));
        r == Ok(())
    }
}

impl Drop for Condvar {
    fn drop(&mut self) {
        let r = unsafe { self.inner.destroy() };
        debug_assert_eq!(r, Ok(()));
    }
}

impl Default for Condvar {
    fn default() -> Condvar {
        Condvar::new()
    }
}
