const std = @import("std");
const time = std.time;
const Timer = time.Timer;

pub fn main() !void {
    var file = try std.fs.cwd()
        .openFile("../input.txt", .{});
    defer file.close();
    var buf: [32 * 1024]u8 = undefined;
    buf[try file.read(&buf) - 1] = ' ';
    // _ = try file.read(&buf);

    var timer = try Timer.start();

    var t: u32 = 0;
    var split = std.mem.splitScalar(u8, &buf, '\n');
    out: while (split.next()) |line| {
        // if (line[0] == 0) continue;
        var split_iter = std.mem.split(u8, line, " ");

        var dir: i8 = 0;
        var prev = try std.fmt.parseInt(i8, split_iter.next().?, 10);
        while (split_iter.next()) |strnum| {
            const n = try std.fmt.parseInt(i8, strnum, 10);
            const d = prev - n;
            if (d == 0 or @abs(d) > 3)
                continue :out;

            if (dir == 0) {
                dir = d;
            } else if (d ^ dir < 0)
                continue :out;
            prev = n;
        }

        t += 1;
    }

    const elapsed: f64 = @floatFromInt(timer.read());
    std.debug.print("Total {d} in {d:.6}µs\n", .{ t, elapsed / time.ns_per_us });
}
