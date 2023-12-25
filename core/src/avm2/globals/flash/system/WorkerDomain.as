package flash.system {
    import flash.utils.ByteArray;
    
    public final class WorkerDomain {
        public static const isSupported: Boolean = false;

        public function WorkerDomain() {
            throw new ArgumentError("Error #2012: WorkerDomain$ class cannot be instantiated.", 2012)
        }

        public native function createWorker(swf:ByteArray, giveAppPrivileges:Boolean = false):Worker
    }
}