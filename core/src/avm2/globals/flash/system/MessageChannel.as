package flash.system {
    import flash.events.EventDispatcher;
    
    public final class MessageChannel extends EventDispatcher {
        public native function send(arg:*, queueLimit:int = -1):void;
    }
}