package  {
	
	import flash.display.MovieClip;
	
	
	public class MyHitState extends MovieClip {
		
		
		public function MyHitState() {
			trace("MyHitState constructor called!");
			new EventWatcher(this);
		}
	}
	
}
