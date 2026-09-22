---@class GameplayDemo : epok.Actor3D
local GameplayDemo = epok.Actor3D:extend()

GameplayDemo.frames = epok.Hidden(0)
GameplayDemo.was_paused = epok.Hidden(false)
GameplayDemo.resource_slots = epok.Hidden(epok.UInt32(0))
GameplayDemo.vertex_query_failed = epok.Hidden(false)
GameplayDemo.target = epok.ActorRef(epok.Actor3D)
GameplayDemo.mesh = epok.ComponentRef(epok.Mesh3DComponent)
GameplayDemo.audio = epok.ComponentRef(epok.AudioComponent)
GameplayDemo.particles = epok.ComponentRef(epok.ParticleEmitterComponent)
GameplayDemo.timeline = epok.ComponentRef(epok.TimelineComponent)
GameplayDemo.effect = epok.ComponentRef(epok.ParticleEffectComponent)

---@override
function GameplayDemo:begin_play()
    local clock = epok.time.snapshot()
    self.was_paused = clock.paused
    epok.memory_card.clear_staged_payload()
    epok.memory_card.set_staged_word(0, 131073)
    epok.memory_card.set_staged_word(1, 1196445509)
    local resources = epok.resources.snapshot()
    self.resource_slots = resources.slot_capacity
    local transition = epok.scene.transition_snapshot()
    self.was_paused = self.was_paused or transition.busy
    -- Scene-global distance fog through the same group. Handing the current
    -- record straight back is the round trip the setter has to accept, so a
    -- false result here is a real failure rather than a rejected range.
    local fog = epok.scene.fog()
    self.was_paused = self.was_paused or not epok.scene.set_fog(fog)
    -- The post-HUD screen fade through the same group. It clamps rather than
    -- rejecting, so the round trip is read back instead of tested on a result:
    -- 300 saturates to 255 and the demo hands the display back clear.
    epok.scene.set_screen_fade(300)
    local faded = epok.scene.screen_fade()
    epok.scene.set_screen_fade(0)
    self.was_paused = self.was_paused or faded ~= 255 or epok.scene.screen_fade() ~= 0
    local probe = epok.math.vector3(1.0, 2.0, 3.0)
    local doubled = epok.math.add(probe, probe)
    self.frames = self.frames + epok.to_int(doubled.x)
    local tween = epok.utilities.tween_start(0.0, 1.0, 1.0, 0)
    local tween_step = epok.utilities.tween_advance(tween, 0.5)
    -- The full plan through the same group: a quarter-second wait, an appended
    -- curve, and a two-leg mirrored replay. Enumerators are written as the numbers
    -- they are stored as: 13 is InOutQuint, 16 is InOutCirc, 2 is PingPong.
    local plan = epok.utilities.tween_schedule(0.0, 1.0, 1.0, 13, 0.25, 2, 2)
    local plan_step = epok.utilities.tween_advance(plan, 0.5)
    local eased = epok.utilities.ease(0.5, 16)
    -- One clock, three components: a position or a tint is one call rather than
    -- three, and the state stays owned by this class exactly as the scalar one is.
    local span = epok.utilities.vector_tween_schedule(probe, doubled, 1.0, 15, 0.0, 1, 3)
    local span_step = epok.utilities.vector_tween_advance(span, 0.25)
    local held = epok.utilities.vector_tween_value(epok.utilities.vector_tween_cancel(span_step.state))
    self.frames = self.frames + epok.to_int(eased) + epok.to_int(held.z)
    local queue = epok.utilities.event_queue_clear()
    local emitted = epok.utilities.event_queue_emit(queue, 7, 42, self.target)
    local polled = epok.utilities.event_queue_poll(emitted.state)
    self.was_paused = self.was_paused or tween_step.completed or not polled.valid
    self.was_paused = self.was_paused or plan_step.completed or span_step.completed

    -- Typed foreign receivers use the same operation catalog as Blueprint.
    self.audio:set_volume(0.5)
    self.audio:set_pitch(1.0)
    self.particles:set_rate(4.0)
    self.particles:burst(4)
    local vertex = self.mesh:sample_vertex(0, 0, 0)
    self.vertex_query_failed = not vertex.success
    local geometry = self.mesh:sample_geometry_vertex(0, 0)
    local geometry_state = self.mesh:geometry_state()
    if geometry_state == 1 then
        self.mesh:request_geometry()
    end
    self.vertex_query_failed = self.vertex_query_failed or not geometry.success

    -- These deliberately unconfigured playback components exercise the
    -- bounded invalid-handle path without requiring project-authored C++.
    local sequence = epok.playback.play_sequence(self.timeline)
    local sequence_state = epok.playback.sequence_state(sequence)
    local effect = epok.playback.play_effect(self.effect)
    local effect_state = epok.playback.effect_state(effect)
    self.was_paused = self.was_paused or sequence_state.revision > 0 or effect_state.revision > 0

    epok.focus.clear()
    epok.focus.add(self.target)
    local focus = epok.focus.snapshot()
    if focus.valid then
        self.target:set_active(false)
        self.target:set_active(true)
    end
end

---@override
function GameplayDemo:on_frame(frame_microseconds)
    self.frames = self.frames + 1
    if self.frames == 1 then
        epok.time.set_paused(true)
    elseif self.frames == 2 then
        epok.time.set_paused(false)
    end
end

return GameplayDemo
