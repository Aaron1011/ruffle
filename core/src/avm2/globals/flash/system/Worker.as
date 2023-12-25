package flash.system {
    [Ruffle(InstanceAllocator)]
    public final class Worker {
        public function Worker() {
            throw new ArgumentError("Error #2012: Worker$ class cannot be instantiated.", 2012);
        }

        public static function get isSupported():Boolean {
            return true;
        }

        public static native function get current():Worker;
        public static native function get isPrimordial():Boolean
    }
}
