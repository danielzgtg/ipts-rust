use crate::ipts::inner::IptsImpl;
use crate::DeviceInfo;

pub trait IptsAccess: Sized + 'static {
    fn ipts_impl(&self) -> &IptsImpl;

    fn ipts_impl_mut(&mut self) -> &mut IptsImpl;

    fn ipts_impl_destructure(self) -> IptsImpl;
}

pub trait IptsExt: Sized {
    fn send_reset(self);

    fn device_info(&self) -> &DeviceInfo;

    fn read(&self, buf: &mut [u8; 16384]);
}

impl<T> IptsExt for T
where
    T: IptsAccess,
{
    fn send_reset(self) {
        // Async: Since this consumes self,
        // naturally the caller would call this outside an async context
        self.ipts_impl_destructure().send_reset();
    }

    fn device_info(&self) -> &DeviceInfo {
        // Async safe: Reads a reference only
        self.ipts_impl().device_info()
    }

    fn read(&self, buf: &mut [u8; 16384]) {
        // Async safe: This should just be a memcpy from the kernel to userspace,
        // and that shouldn't ever block
        self.ipts_impl().read(buf);
    }
}
