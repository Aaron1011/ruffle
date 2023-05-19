package  {
	
	import flash.display.MovieClip;
	
	
	public class SideObserver extends MovieClip {
		
		
		public function SideObserver() {
			trace("Constructed SideObserver");
			new EventWatcher(this);
		}
	}
	
}
