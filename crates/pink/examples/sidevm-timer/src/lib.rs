mod sleep {
    use pink_sidevm_env::{self as env, ocall_funcs_guest as ocall};

    use core::pin::Pin;
    use std::future::Future;
    use std::task::{Context, Poll};
    use std::time::Duration;
    /// Resource ID. Think of it as a FD.
    pub struct ResourceId(pub i32);

    impl Drop for ResourceId {
        fn drop(&mut self) {
            let _ = ocall::close(self.0);
        }
    }

    pub struct Sleep {
        id: ResourceId,
    }

    pub fn sleep(duration: Duration) -> Sleep {
        let id = ocall::create_timer(duration.as_millis() as i32).expect("failed to create timer");
        Sleep { id: ResourceId(id) }
    }

    impl Future for Sleep {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let rv = ocall::poll(self.id.0).expect("Poll timer failed");
            match rv {
                env::Poll::Ready(_) => Poll::Ready(()),
                env::Poll::Pending => Poll::Pending,
            }
        }
    }
}

#[pink_sidevm_env::main]
async fn main() {
    use pink_sidevm_env::ocall_funcs_guest as ocall;
    use std::time::Duration;

    ocall::enable_ocall_trace(true).unwrap();
    assert_eq!(ocall::echo(vec![4, 2]).unwrap(), vec![4, 2]);
    sleep::sleep(Duration::from_secs(3)).await
}
